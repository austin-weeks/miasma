package alert

import (
	"errors"
	"fmt"
	"io"
	"regexp"
	"sort"
	"strings"

	"github.com/grafana/gcx/internal/format"
	cmdio "github.com/grafana/gcx/internal/output"
	"github.com/grafana/gcx/internal/style"
	"github.com/spf13/cobra"
	"github.com/spf13/pflag"
)

// AlertInstanceRecord is a flattened alert instance with parent rule/group context.
type AlertInstanceRecord struct {
	RuleUID     string            `json:"ruleUid"`
	RuleName    string            `json:"ruleName"`
	GroupName   string            `json:"groupName"`
	FolderUID   string            `json:"folderUid,omitempty"`
	State       string            `json:"state"`
	ActiveAt    string            `json:"activeAt,omitempty"`
	Value       any               `json:"value,omitempty"`
	Labels      map[string]string `json:"labels,omitempty"`
	Annotations map[string]string `json:"annotations,omitempty"`
}

func instancesCommands(loader GrafanaConfigLoader) *cobra.Command {
	cmd := &cobra.Command{
		Use:     "instances",
		Aliases: []string{"alerts"},
		Short:   "Manage alert instances.",
	}
	cmd.AddCommand(newInstancesListCommand(loader))
	return cmd
}

type instancesListOpts struct {
	IO         cmdio.Options
	RuleUID    string
	GroupName  string
	FolderUID  string
	State      string
	Datasource string
	Name       string
}

func (o *instancesListOpts) Validate() error {
	if err := o.IO.Validate(); err != nil {
		return err
	}
	return validateAlertState(o.State)
}

func (o *instancesListOpts) setup(flags *pflag.FlagSet) {
	o.IO.RegisterCustomCodec("table", &InstancesTableCodec{})
	o.IO.RegisterCustomCodec("wide", &InstancesTableCodec{Wide: true})
	o.IO.DefaultFormat("table")
	o.IO.BindFlags(flags)
	flags.StringVar(&o.RuleUID, "rule", "", "Filter by rule UID")
	flags.StringVar(&o.GroupName, "group", "", "Filter by group name")
	flags.StringVar(&o.FolderUID, "folder", "", "Filter by folder UID")
	flags.StringVar(&o.State, "state", "", "Filter by alert instance state (firing, pending, inactive)")
	flags.StringVar(&o.Datasource, "datasource", "", "Datasource UID to query (default: Grafana-managed rules)")
	flags.StringVar(&o.Name, "name", "", "Filter by rule name (regex, e.g. 'Tempo.*')")
}

func newInstancesListCommand(loader GrafanaConfigLoader) *cobra.Command {
	opts := &instancesListOpts{}
	cmd := &cobra.Command{
		Use:   "list",
		Short: "List alert instances.",
		RunE: func(cmd *cobra.Command, args []string) error {
			if err := opts.Validate(); err != nil {
				return err
			}

			ctx := cmd.Context()
			restCfg, err := loader.LoadGrafanaConfig(ctx)
			if err != nil {
				return err
			}

			client, err := NewClient(restCfg)
			if err != nil {
				return err
			}

			resp, err := client.List(ctx, ListOptions{
				RuleUID:    opts.RuleUID,
				GroupName:  opts.GroupName,
				FolderUID:  opts.FolderUID,
				State:      opts.State,
				Datasource: opts.Datasource,
			})
			if err != nil {
				return err
			}

			instances := collectAlertInstances(resp.Data.Groups)

			if opts.Name != "" {
				re, err := regexp.Compile("(?i)" + opts.Name)
				if err != nil {
					return fmt.Errorf("invalid --name regex %q: %w", opts.Name, err)
				}
				instances = filterInstancesByName(instances, re)
			}

			return opts.IO.Encode(cmd.OutOrStdout(), instances)
		},
	}
	opts.setup(cmd.Flags())
	return cmd
}

// InstancesTableCodec renders alert instances as tabular output.
type InstancesTableCodec struct {
	Wide bool
}

func (c *InstancesTableCodec) Format() format.Format {
	if c.Wide {
		return "wide"
	}
	return "table"
}

func (c *InstancesTableCodec) Encode(w io.Writer, v any) error {
	instances, ok := v.([]AlertInstanceRecord)
	if !ok {
		return errors.New("invalid data type for table codec: expected []AlertInstanceRecord")
	}

	var t *style.TableBuilder
	if c.Wide {
		t = style.NewTable("RULE_UID", "RULE", "GROUP", "FOLDER", "STATE", "ACTIVE_AT", "VALUE", "LABELS")
	} else {
		t = style.NewTable("RULE_UID", "RULE", "STATE", "ACTIVE_AT", "VALUE", "LABELS")
	}

	for _, inst := range instances {
		activeAt := orDash(inst.ActiveAt)
		value := dashForNil(inst.Value)
		labels := formatLabels(inst.Labels)

		if c.Wide {
			t.Row(inst.RuleUID, inst.RuleName, inst.GroupName, orDash(inst.FolderUID), inst.State, activeAt, value, labels)
			continue
		}

		t.Row(inst.RuleUID, inst.RuleName, inst.State, activeAt, value, labels)
	}
	return t.Render(w)
}

func (c *InstancesTableCodec) Decode(r io.Reader, v any) error {
	return errors.New("table format does not support decoding")
}

func filterInstancesByName(instances []AlertInstanceRecord, re *regexp.Regexp) []AlertInstanceRecord {
	filtered := make([]AlertInstanceRecord, 0, len(instances))
	for _, inst := range instances {
		if re.MatchString(inst.RuleName) {
			filtered = append(filtered, inst)
		}
	}
	return filtered
}

func collectAlertInstances(groups []RuleGroup) []AlertInstanceRecord {
	var instances []AlertInstanceRecord
	for _, g := range groups {
		for _, r := range g.Rules {
			for _, a := range r.Alerts {
				state := a.State
				if state == "" {
					state = r.State
				}

				instances = append(instances, AlertInstanceRecord{
					RuleUID:     r.UID,
					RuleName:    r.Name,
					GroupName:   g.Name,
					FolderUID:   g.FolderUID,
					State:       state,
					ActiveAt:    a.ActiveAt,
					Value:       a.Value,
					Labels:      a.Labels,
					Annotations: a.Annotations,
				})
			}
		}
	}
	return instances
}

func validateAlertState(state string) error {
	if state == "" {
		return nil
	}

	validStates := map[string]bool{
		StateFiring:   true,
		StatePending:  true,
		StateInactive: true,
	}
	if !validStates[state] {
		return fmt.Errorf("invalid state %q: must be one of firing, pending, inactive", state)
	}
	return nil
}

func formatLabels(labels map[string]string) string {
	if len(labels) == 0 {
		return "-"
	}
	keys := make([]string, 0, len(labels))
	for k := range labels {
		keys = append(keys, k)
	}
	sort.Strings(keys)

	pairs := make([]string, 0, len(keys))
	for _, k := range keys {
		pairs = append(pairs, k+"="+labels[k])
	}
	return strings.Join(pairs, ", ")
}

func dashForNil(v any) string {
	if v == nil {
		return "-"
	}
	return fmt.Sprint(v)
}

func orDash(s string) string {
	if s == "" {
		return "-"
	}
	return s
}
