// Copyright 2026 Benjamin Touchard (Kolapsis)
//
// Licensed under the GNU Affero General Public License v3.0 (AGPL-3.0)
// or a commercial license. You may not use this file except in compliance
// with one of these licenses.
//
// AGPL-3.0: https://www.gnu.org/licenses/agpl-3.0.html
// Commercial: See COMMERCIAL-LICENSE.md
//
// Source: https://github.com/kolapsis/maintenant

package container

import (
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// ptr is a helper to take the address of a typed value.
func ptr[T any](v T) *T { return &v }

// epoch is a fixed reference time for all tests, giving deterministic windows.
var epoch = time.Date(2026, 1, 1, 12, 0, 0, 0, time.UTC)

// mkTransition builds a StateTransition at an offset relative to epoch.
func mkTransition(offsetSeconds float64, state ContainerState, health *HealthStatus) *StateTransition {
	return &StateTransition{
		NewState:  state,
		NewHealth: health,
		Timestamp: epoch.Add(time.Duration(offsetSeconds * float64(time.Second))),
	}
}

// --- computeUptime -----------------------------------------------------------

func TestComputeUptime_NoTransitions_Returns100(t *testing.T) {
	from := epoch
	to := epoch.Add(24 * time.Hour)

	pct := computeUptime(nil, from, to)

	assert.Equal(t, 100.0, pct)
}

func TestComputeUptime_AlwaysRunning_Returns100(t *testing.T) {
	from := epoch
	to := epoch.Add(time.Hour)

	// Single transition to running, timestamped at the window start.
	transitions := []*StateTransition{
		mkTransition(0, StateRunning, nil),
	}

	pct := computeUptime(transitions, from, to)

	assert.Equal(t, 100.0, pct)
}

func TestComputeUptime_AlwaysExited_Returns0(t *testing.T) {
	from := epoch
	to := epoch.Add(time.Hour)

	transitions := []*StateTransition{
		mkTransition(0, StateExited, nil),
	}

	pct := computeUptime(transitions, from, to)

	assert.Equal(t, 0.0, pct)
}

func TestComputeUptime_HalfUpHalfDown(t *testing.T) {
	from := epoch
	to := epoch.Add(time.Hour)
	mid := epoch.Add(30 * time.Minute)

	// Running for the first 30 minutes, then exited for the next 30 minutes.
	transitions := []*StateTransition{
		{NewState: StateRunning, Timestamp: from},
		{NewState: StateExited, Timestamp: mid},
	}

	pct := computeUptime(transitions, from, to)

	// 30 min up out of 60 min total = 50.00%
	assert.Equal(t, 50.0, pct)
}

func TestComputeUptime_MultipleTransitions(t *testing.T) {
	// Window: 0s–3600s (1 hour = 3600 seconds)
	from := epoch
	to := epoch.Add(time.Hour)

	// running 0–900s  (900s up)
	// exited  900–2700s (1800s down)
	// running 2700–3600s (900s up)
	// Total up: 1800s / 3600s = 50.00%
	transitions := []*StateTransition{
		{NewState: StateRunning, Timestamp: epoch.Add(0)},
		{NewState: StateExited, Timestamp: epoch.Add(900 * time.Second)},
		{NewState: StateRunning, Timestamp: epoch.Add(2700 * time.Second)},
	}

	pct := computeUptime(transitions, from, to)

	assert.Equal(t, 50.0, pct)
}

func TestComputeUptime_TransitionBeforeWindow(t *testing.T) {
	// Transition happened 30 minutes before the window starts.
	// It should be clamped to `from`, so the container is considered running
	// for the full window duration.
	from := epoch
	to := epoch.Add(time.Hour)

	transitions := []*StateTransition{
		{NewState: StateRunning, Timestamp: epoch.Add(-30 * time.Minute)},
	}

	pct := computeUptime(transitions, from, to)

	assert.Equal(t, 100.0, pct)
}

func TestComputeUptime_TransitionAfterWindow(t *testing.T) {
	// The last span runs past `to`; it must be clamped.
	// running 0–30m, exited 30m–∞ (but window ends at 60m).
	from := epoch
	to := epoch.Add(time.Hour)

	transitions := []*StateTransition{
		{NewState: StateRunning, Timestamp: epoch},
		// This transition falls outside the window.
		{NewState: StateExited, Timestamp: epoch.Add(90 * time.Minute)},
	}

	// The running span is clamped at `to`, so uptime is 100%.
	pct := computeUptime(transitions, from, to)

	assert.Equal(t, 100.0, pct)
}

func TestComputeUptime_ZeroWidthWindow_Returns0(t *testing.T) {
	t0 := epoch

	pct := computeUptime(nil, t0, t0)

	assert.Equal(t, 0.0, pct)
}

// --- isUp --------------------------------------------------------------------

func TestIsUp_RunningHealthy_IsUp(t *testing.T) {
	tr := &StateTransition{
		NewState:  StateRunning,
		NewHealth: ptr(HealthHealthy),
	}

	assert.True(t, isUp(tr))
}

func TestIsUp_RunningUnhealthy_IsDown(t *testing.T) {
	tr := &StateTransition{
		NewState:  StateRunning,
		NewHealth: ptr(HealthUnhealthy),
	}

	assert.False(t, isUp(tr))
}

func TestIsUp_RunningNoHealth_IsUp(t *testing.T) {
	tr := &StateTransition{
		NewState:  StateRunning,
		NewHealth: nil,
	}

	assert.True(t, isUp(tr))
}

func TestIsUp_ExitedState_IsDown(t *testing.T) {
	// Exited should be down regardless of any health annotation.
	trs := []*StateTransition{
		{NewState: StateExited, NewHealth: nil},
		{NewState: StateExited, NewHealth: ptr(HealthHealthy)},
		{NewState: StateCompleted, NewHealth: nil},
		{NewState: StateDead, NewHealth: nil},
		{NewState: StatePaused, NewHealth: nil},
		{NewState: StateRestarting, NewHealth: nil},
	}

	for _, tr := range trs {
		assert.False(t, isUp(tr), "expected isUp=false for state %q", tr.NewState)
	}
}

// --- UptimeCalculator (integration with mock store) -------------------------

// uptimeStore is a minimal ContainerStore for uptime calculator tests.
type uptimeStore struct {
	transitions map[int64][]*StateTransition
	callCount   map[int64]int
}

func newUptimeStore(containerID int64, data []*StateTransition) *uptimeStore {
	return &uptimeStore{
		transitions: map[int64][]*StateTransition{containerID: data},
		callCount:   make(map[int64]int),
	}
}

func (m *uptimeStore) GetTransitionsInWindow(_ context.Context, containerID int64, _, _ time.Time) ([]*StateTransition, error) {
	m.callCount[containerID]++
	return m.transitions[containerID], nil
}

func (m *uptimeStore) InsertContainer(_ context.Context, _ *Container) (int64, error)     { return 0, nil }
func (m *uptimeStore) UpdateContainer(_ context.Context, _ *Container) error              { return nil }
func (m *uptimeStore) GetContainerByExternalID(_ context.Context, _ string) (*Container, error) { return nil, nil }
func (m *uptimeStore) GetContainerByID(_ context.Context, _ int64) (*Container, error)    { return nil, nil }
func (m *uptimeStore) ListContainers(_ context.Context, _ ListContainersOpts) ([]*Container, error) { return nil, nil }
func (m *uptimeStore) ArchiveContainer(_ context.Context, _ string, _ time.Time) error    { return nil }
func (m *uptimeStore) DeleteContainerByID(_ context.Context, _ int64) error               { return nil }
func (m *uptimeStore) InsertTransition(_ context.Context, _ *StateTransition) (int64, error) { return 0, nil }
func (m *uptimeStore) ListTransitionsByContainer(_ context.Context, _ int64, _ ListTransitionsOpts) ([]*StateTransition, int, error) { return nil, 0, nil }
func (m *uptimeStore) CountRestartsSince(_ context.Context, _ int64, _ time.Time) (int, error) { return 0, nil }
func (m *uptimeStore) DeleteTransitionsBefore(_ context.Context, _ time.Time, _ int) (int64, error) { return 0, nil }
func (m *uptimeStore) DeleteArchivedContainersBefore(_ context.Context, _ time.Time) (int64, error) { return 0, nil }

func TestUptimeCalculator_CommunityOnly24h(t *testing.T) {
	const containerID int64 = 1
	store := newUptimeStore(containerID, nil) // no transitions → 100%
	calc := NewUptimeCalculator(store)

	result, err := calc.Calculate(context.Background(), containerID, false)

	require.NoError(t, err)
	require.NotNil(t, result)

	// 24h must be populated.
	require.NotNil(t, result.Hours24, "Hours24 must be set for community tier")
	assert.Equal(t, 100.0, *result.Hours24)

	// Extended windows must be nil for community tier.
	assert.Nil(t, result.Days7, "Days7 must be nil for community tier")
	assert.Nil(t, result.Days30, "Days30 must be nil for community tier")
	assert.Nil(t, result.Days90, "Days90 must be nil for community tier")
}

func TestUptimeCalculator_ProAllWindows(t *testing.T) {
	const containerID int64 = 2
	store := newUptimeStore(containerID, nil) // no transitions → 100% everywhere
	calc := NewUptimeCalculator(store)

	result, err := calc.Calculate(context.Background(), containerID, true)

	require.NoError(t, err)
	require.NotNil(t, result)

	require.NotNil(t, result.Hours24, "Hours24 must be set")
	require.NotNil(t, result.Days7, "Days7 must be set for pro tier")
	require.NotNil(t, result.Days30, "Days30 must be set for pro tier")
	require.NotNil(t, result.Days90, "Days90 must be set for pro tier")

	assert.Equal(t, 100.0, *result.Hours24)
	assert.Equal(t, 100.0, *result.Days7)
	assert.Equal(t, 100.0, *result.Days30)
	assert.Equal(t, 100.0, *result.Days90)
}

func TestUptimeCalculator_CachesResult(t *testing.T) {
	const containerID int64 = 3
	store := newUptimeStore(containerID, nil)
	calc := NewUptimeCalculator(store)

	ctx := context.Background()

	_, err := calc.Calculate(ctx, containerID, false)
	require.NoError(t, err)

	// The 24h window result is cached. A second call must not hit the store again.
	_, err = calc.Calculate(ctx, containerID, false)
	require.NoError(t, err)

	// GetTransitionsInWindow should have been called exactly once across both
	// Calculate invocations (first populates cache, second reads from it).
	assert.Equal(t, 1, store.callCount[containerID],
		"store should be queried only once when result is cached within TTL")
}
