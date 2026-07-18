use crate::response_templates::research::slot::Slot;

pub const CODE_HEADINGS: &[&str] = &[
    "Core Implementation",
    "Novel Approach",
    "Implementation Details",
    "Methodological Framework",
    "Key Components",
    "Technical Architecture",
    "Solution Overview",
    "Implementation Strategy",
    "Data Structure Specification",
    "Structural Design",
    "Implementation Approach",
    "Algorithmic Formulation",
    "Implementation Methodology",
    "System Architecture",
    "Design Rationale",
    "Operational Semantics",
    "Dataflow And Control Structure",
    "Component Interaction Model",
    "Computational Pipeline",
    "State Transition Strategy",
    "Constraint Encoding",
    "Correctness-Preserving Design",
    "Extensibility Analysis",
    "Performance Considerations",
    "Implementation Analysis",
    "Practical Realization",
];

pub const CODE_INTRODUCTIONS: &[&[Slot]] = &[
    &[Slot::Str(
        "The implementation is organized to make the principal execution path explicit while preserving \
        rigorous treatment of boundary conditions. \
        The code that follows reflects this ordering directly, so structural choices correspond closely to semantic intent. \
        This arrangement supports both interpretability and correctness analysis.",
    )],
    &[
        Slot::Str("Our treatment of "),
        Slot::Topic,
        Slot::Str(
            " prioritizes a minimal, verifiable baseline before introducing additional mechanism. \
            Consequently, core operations remain concise, while exceptional behavior is isolated in well-scoped branches. \
            The resulting form is straightforward to evaluate, test, and extend.",
        ),
    ],
    &[
        Slot::Str("In order to effectively address "),
        Slot::Topic,
        Slot::Str(
            ", we decompose execution into a sequence of narrowly defined stages. \
            This staging limits cross-component state leakage and improves localization of failure modes. \
            The subsequent implementation is structured around this progression.",
        ),
    ],
    &[
        Slot::Str("For "),
        Slot::Topic,
        Slot::Str(
            ", validation logic is incorporated as a first-order design concern rather than a post hoc addition. \
            Defensive checks appear near interface boundaries, followed by a comparatively linear computational core. \
            This layout preserves readability without weakening guarantees.",
        ),
    ],
    &[
        Slot::Str("The implementation for "),
        Slot::Topic,
        Slot::Str(
            " is best interpreted through component contracts: each unit specifies explicit preconditions and postconditions. \
            Once these contracts are established, control flow and dependency structure become significantly easier to reason about. \
            This contract-centered framing supports maintainable complexity.",
        ),
    ],
    &[
        Slot::Str("Our approach to "),
        Slot::Topic,
        Slot::Str(
            " adopts a single explicit state representation, a predictable transition model, and limited hidden coupling. \
            These decisions preserve analytical clarity while providing a stable foundation for subsequent extension.",
        ),
    ],
    &[
        Slot::Str("Rather than expressing "),
        Slot::Topic,
        Slot::Str(
            " as a single monolithic routine, we organize it as a small collection of cooperating operations. \
            Each operation maintains a sharply delimited responsibility, enabling local reasoning about behavior and invariants. \
            The code below follows this separation closely.",
        ),
    ],
    &[
        Slot::Str("For "),
        Slot::Topic,
        Slot::Str(
            ", subtle ordering dependencies are a principal source of defects, so sequencing is made explicit throughout the implementation. \
            Preconditions appear at the point of use rather than as implicit assumptions. \
            This strategy improves predictability across both nominal and atypical inputs.",
        ),
    ],
    &[
        Slot::Str("The code path for "),
        Slot::Topic,
        Slot::Str(
            " follows a three-phase structure: normalization, transformation, and finalization. \
            This decomposition yields clear insertion points for validation, monitoring, and refinement. \
            The structure also improves interpretability for later contributors.",
        ),
    ],
    &[
        Slot::Str("In constructing "),
        Slot::Topic,
        Slot::Str(
            ", design tradeoffs are encoded directly in control structure rather than relegated to prose. \
            Primary paths, fallback paths, and exception paths are each represented explicitly. \
            This representation makes engineering priorities inspectable.",
        ),
    ],
    &[
        Slot::Str("For "),
        Slot::Topic,
        Slot::Str(
            ", the central decision is to make data movement explicit at each interface boundary. \
            Inputs are normalized once, transformed through deliberate stages, and emitted under a consistent output contract. \
            This data-centric framing simplifies both auditing and verification.",
        ),
    ],
    &[
        Slot::Str("The implementation strategy for "),
        Slot::Topic,
        Slot::Str(
            " emphasizes composable modules rather than a single large procedure. \
            Units with explicit interfaces support fine-grained testing and controlled reuse where abstraction boundaries align.",
        ),
    ],
    &[
        Slot::Str("In this formulation of "),
        Slot::Topic,
        Slot::Str(
            ", we prioritize semantic clarity before optimization. \
            Once behavioral correctness is straightforward to reason about, targeted performance adjustments can be introduced with lower risk and clearer justification.",
        ),
    ],
    &[
        Slot::Str("State management in "),
        Slot::Topic,
        Slot::Str(
            " is made explicit through well-defined transitions and containment of side effects at subsystem boundaries. \
            This approach reduces hidden coupling and facilitates diagnosis when behavior deviates from expectation.",
        ),
    ],
    &[
        Slot::Str("To keep "),
        Slot::Topic,
        Slot::Str(
            " tractable, low-level mechanical details are encapsulated behind narrowly scoped abstractions with explicit naming. \
            Call sites therefore express intent-level operations, while bookkeeping remains localized to lower layers.",
        ),
    ],
    &[
        Slot::Str("In our implementation of "),
        Slot::Topic,
        Slot::Str(
            ", component boundaries serve as the primary enforcement points for safety assumptions. \
            Each handoff validates local preconditions before passing control onward, which limits error propagation and simplifies fault attribution.",
        ),
    ],
    &[
        Slot::Str("Because requirements for "),
        Slot::Topic,
        Slot::Str(
            " often evolve, extensibility points are made explicit in the current design. \
            Variable behavior is isolated behind stable interfaces, enabling incremental expansion without destabilizing established pathways.",
        ),
    ],
    &[
        Slot::Str("A central objective in "),
        Slot::Topic,
        Slot::Str(
            " is to align operation placement with semantic role in the execution flow. \
            When structure communicates meaning directly, misuse decreases and formal review becomes more reliable.",
        ),
    ],
    &[
        Slot::Str("In "),
        Slot::Topic,
        Slot::Str(
            ", information flow dominates local algorithmic detail as the principal determinant of correctness. \
            The implementation therefore emphasizes traceability from input normalization through intermediate transformation to output generation.",
        ),
    ],
    &[
        Slot::Str("A key design insight for "),
        Slot::Topic,
        Slot::Str(
            " is the separation of decisions resolvable statically from those requiring runtime context. \
            The implementation below preserves this distinction, reducing overhead on primary paths while containing dynamic logic.",
        ),
    ],
    &[
        Slot::Str("In practical deployments, "),
        Slot::Topic,
        Slot::Str(
            " performs best when nominal cases remain efficient and edge conditions remain explicit. \
            The following implementation preserves this balance by keeping common paths concise while handling boundary behavior through deliberate, structured branches.",
        ),
    ],
    &[
        Slot::Str("Our implementation of "),
        Slot::Topic,
        Slot::Str(
            " is organized around a concise set of invariants preserved by every execution path. \
            Maintaining these invariants as first-class design constraints simplifies correctness arguments and improves regression detection.",
        ),
    ],
    &[
        Slot::Str("In this realization of "),
        Slot::Topic,
        Slot::Str(
            ", we favor maintainable structure over maximal theoretical compactness. \
            The resulting code aligns with practical engineering constraints while preserving the guarantees required by the underlying model.",
        ),
    ],
    &[
        Slot::Str("A substantial portion of reliability in "),
        Slot::Topic,
        Slot::Str(
            " derives from representational clarity rather than logic alone. \
            The implementation uses semantically precise naming and narrowly scoped helper roles, allowing intent to be inferred directly from structure.",
        ),
    ],
    &[
        Slot::Str("The architecture for "),
        Slot::Topic,
        Slot::Str(
            " is grounded in a minimal set of primitive operations from which higher-order behavior is composed. \
            This strategy keeps the foundational layer small and verifiable while preserving expressive capability at higher layers.",
        ),
    ],
    &[
        Slot::Str("In designing "),
        Slot::Topic,
        Slot::Str(
            ", failure modes are modeled as first-class behaviors rather than exceptional afterthoughts. \
            Instead of ad hoc checks, the implementation applies a consistent error discipline that preserves graceful degradation and diagnostic clarity.",
        ),
    ],
    &[
        Slot::Str("The strategy for "),
        Slot::Topic,
        Slot::Str(
            " is to preclude invalid states at the earliest representational boundary and make valid transitions straightforward to express. \
            This reduces defensive branching in the computational core and eliminates broad classes of avoidable defects.",
        ),
    ],
    &[
        Slot::Str("Implementing "),
        Slot::Topic,
        Slot::Str(
            " effectively requires explicit separation between hard constraints and legitimate degrees of freedom. \
            The code encodes this distinction directly, supporting lawful variation while ruling out invalid combinations by construction.",
        ),
    ],
    &[
        Slot::Str("In formalizing "),
        Slot::Topic,
        Slot::Str(
            ", we prioritize explicit invariants over implicit convention. \
            The implementation therefore places validation and transition logic at points where correctness obligations are easiest to inspect. \
            This structure supports both proof-oriented reasoning and practical maintenance.",
        ),
    ],
    &[
        Slot::Str("The implementation presented for "),
        Slot::Topic,
        Slot::Str(
            " adopts a layered construction in which representation, transformation, and orchestration are separated by clear interfaces. \
            Such separation reduces semantic drift between components and improves the reproducibility of behavior under perturbation. \
            The code excerpt below is organized to reflect that analytical boundary.",
        ),
    ],
    &[
        Slot::Str("For "),
        Slot::Topic,
        Slot::Str(
            ", the principal architectural choice is to encode constraints close to data entry points and preserve normalized structure thereafter. \
            This reduces downstream branching and limits the propagation of malformed state through the pipeline. \
            The resulting control structure remains compact while retaining strong safety properties.",
        ),
    ],
    &[
        Slot::Str("A defining feature of our "),
        Slot::Topic,
        Slot::Str(
            " implementation is the explicit coupling of abstraction boundaries to verification boundaries. \
            Each module is evaluated against local obligations before composition at higher levels of the stack. \
            This design yields predictable behavior while preserving extensibility for future variants.",
        ),
    ],
];
