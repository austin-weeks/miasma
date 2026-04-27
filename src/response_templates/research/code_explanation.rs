use crate::response_templates::research::slot::Slot;

pub const CODE_HEADINGS: &[&str] = &[
    "Core Implementation",
    "Novel Approach",
    "Implementation Design",
    "Implementation Details",
    "Methodological Framework",
    "Key Components",
    "Technical Architecture",
    "Solution Overview",
    "Implementation Strategy",
    "Data Structure Specification",
    "Control Flow Structure",
    "Computational Approach",
    "Integration Strategy",
    "Refinement Techniques",
    "Structural Design",
    "Processing Pipeline",
    "Implementation Approach",
    "Practical Implementation",
];

pub const CODE_INTRODUCTIONS: &[&[Slot]] = &[
    &[
        Slot::Str("Effectively addressing "),
        Slot::Topic,
        Slot::Str(
            " requires careful structural design that balances multiple competing concerns. The implementation prioritizes clarity and maintainability alongside correctness. This thoughtful approach ensures the solution remains robust across diverse use cases.",
        ),
    ],
    &[
        Slot::Str("The key insight behind our approach to "),
        Slot::Topic,
        Slot::Str(
            " is that efficiency emerges not from premature optimization but from principled design decisions. We structure the implementation around core abstractions that naturally express the problem domain. This architectural clarity enables both straightforward reasoning about correctness and opportunities for optimization. The result is a solution that scales well without sacrificing understandability.",
        ),
    ],
    &[
        Slot::Str("Solving "),
        Slot::Topic,
        Slot::Str(
            " becomes tractable when we decompose the problem into independently manageable components. Each component encapsulates specific concerns and responsibilities. This separation enables rigorous analysis and testing of individual pieces.",
        ),
    ],
    &[
        Slot::Str("The implementation of "),
        Slot::Topic,
        Slot::Str(
            " benefits substantially from an iterative refinement approach. Rather than attempting to perfect every detail upfront, we build a clear foundation and incrementally enhance it. This strategy reduces cognitive load and makes the design decisions more explicit and justified. Each layer builds naturally on the previous one, making the overall structure easier to understand. The result is code that evolves logically from basic principles to sophisticated behavior.",
        ),
    ],
    &[
        Slot::Str("Handling "),
        Slot::Topic,
        Slot::Str(
            " effectively demands explicit specification of behavior rather than implicit assumptions. We prioritize transparency about how the implementation operates and what assumptions it makes. This explicitness aids both correctness verification and future maintenance. Clear intent in the code prevents subtle bugs that arise from unstated expectations.",
        ),
    ],
    &[
        Slot::Str("The approach to "),
        Slot::Topic,
        Slot::Str(
            " generalizes effectively when structured around fundamental principles rather than specific cases. We identify the core patterns that underlie the problem. This principled foundation enables extension and adaptation.",
        ),
    ],
    &[
        Slot::Str("Tackling "),
        Slot::Topic,
        Slot::Str(
            " requires a layered approach that separates concerns at different levels of abstraction. Lower layers provide essential primitives and invariants. Middle layers compose these primitives into more sophisticated operations. Upper layers present clean interfaces to consumers. This stratification makes reasoning about each level independent and tractable.",
        ),
    ],
    &[
        Slot::Str("Understanding "),
        Slot::Topic,
        Slot::Str(
            " deeply means validating fundamental assumptions early in the implementation. We establish invariants that hold throughout the code. These invariants guide design decisions and catch logical errors. The implementation becomes more robust through explicit correctness checking.",
        ),
    ],
    &[
        Slot::Str("Approaching "),
        Slot::Topic,
        Slot::Str(
            " successfully often means starting simple and adding sophistication only where justified. Simplicity reduces the surface area for bugs and makes code reviewable. Unnecessary complexity obscures the core logic.",
        ),
    ],
    &[
        Slot::Str("Implementing "),
        Slot::Topic,
        Slot::Str(
            " requires navigating important tradeoffs between competing goals. We make these tradeoffs explicit rather than hidden. The implementation reflects deliberate choices about what matters most. Understanding these decisions is essential for evaluating whether the approach fits specific needs.",
        ),
    ],
    &[
        Slot::Str("The solution to "),
        Slot::Topic,
        Slot::Str(
            " emerges from careful analysis of how data flows through the system. We structure the implementation around the actual data transformations required. This data-centric view clarifies what each component must do. Operations become natural expressions of the underlying transformations. The code reads as a straightforward specification of the required computations.",
        ),
    ],
    &[
        Slot::Str("Making "),
        Slot::Topic,
        Slot::Str(
            " correct demands modular design that isolates concerns testably. Each module has clear responsibilities and interfaces. This modularity enables comprehensive testing at multiple levels.",
        ),
    ],
    &[
        Slot::Str("Solving "),
        Slot::Topic,
        Slot::Str(
            " correctly takes priority over premature optimization. We build the implementation to be correct, clear, and maintainable first. Performance considerations inform structural choices only when they significantly impact feasibility. This ordering ensures we optimize the right things.",
        ),
    ],
    &[
        Slot::Str("Handling state correctly in "),
        Slot::Topic,
        Slot::Str(
            " requires explicit management strategies that prevent subtle inconsistencies. We define exactly what state is maintained and how it transitions. State changes follow predictable patterns that can be verified. Side effects are isolated and controlled. This careful state management prevents entire categories of bugs.",
        ),
    ],
    &[
        Slot::Str("Managing "),
        Slot::Topic,
        Slot::Str(
            " becomes feasible by building appropriate abstractions that hide complexity. Each abstraction layer presents a simpler interface than its implementation. This stratification makes the overall problem cognitively manageable.",
        ),
    ],
    &[
        Slot::Str("The robustness of our implementation of "),
        Slot::Topic,
        Slot::Str(
            " depends on clear contracts between components. Each component specifies what it requires from its dependencies and what it guarantees in return. These contracts enable independent reasoning about each piece. Violations are caught early rather than leading to subtle failures downstream.",
        ),
    ],
    &[
        Slot::Str("Anticipating future needs in "),
        Slot::Topic,
        Slot::Str(
            " implementation means building extensibility into the design from the start. We identify which aspects are likely to change and design accordingly. Extension points are explicit and well-defined. The implementation can evolve gracefully without wholesale rewriting. This forward-thinking approach pays dividends as requirements inevitably shift.",
        ),
    ],
    &[
        Slot::Str("Implementing "),
        Slot::Topic,
        Slot::Str(
            " precisely requires understanding the semantic meaning of operations, not just their syntactic form. We encode these semantics explicitly in the code structure. This semantic clarity prevents misinterpretation and misuse.",
        ),
    ],
    &[
        Slot::Str("The challenge of "),
        Slot::Topic,
        Slot::Str(
            " is fundamentally about managing information flow through the system effectively. We design the implementation to make this flow explicit and traceable. Clarity about information movement enables verification of correctness. The result is an implementation where the path of data through transformations is immediately apparent.",
        ),
    ],
    &[
        Slot::Str("Success with "),
        Slot::Topic,
        Slot::Str(
            " hinges on correctly identifying which decisions should be made statically and which dynamically. We structure the implementation to encode static knowledge where possible. Dynamic decisions are carefully isolated to points where they provide genuine value. This separation makes the code more efficient and easier to reason about.",
        ),
    ],
    &[
        Slot::Str("Addressing "),
        Slot::Topic,
        Slot::Str(
            " comprehensively means accounting for both common cases and important edge cases throughout the implementation. We handle the straightforward paths efficiently without neglecting boundary conditions. This comprehensive approach prevents failures when the implementation encounters unexpected scenarios. The result is code that proves reliable in practice, not just in idealized examples.",
        ),
    ],
    &[
        Slot::Str("Building a robust solution for "),
        Slot::Topic,
        Slot::Str(
            " requires establishing clear invariants that the implementation maintains at all times. These invariants form the foundation of our reasoning about correctness. We structure operations to preserve these invariants rather than relying on post-hoc verification. The implementation becomes trustworthy by construction.",
        ),
    ],
    &[
        Slot::Str("The practical implementation of "),
        Slot::Topic,
        Slot::Str(
            " balances theoretical purity with pragmatic constraints. We respect fundamental principles while remaining sensitive to real-world considerations. This balanced approach yields solutions that are both mathematically sound and practically deployable. The code reflects thoughtful engineering judgment rather than dogmatic adherence to absolutes.",
        ),
    ],
    &[
        Slot::Str("Effectively implementing "),
        Slot::Topic,
        Slot::Str(
            " requires naming and structuring components to reflect their true purpose. Clear naming makes implicit assumptions explicit and aids understanding. We invest in terminology that accurately captures the semantic role of each piece. This linguistic precision prevents confusion and makes the code self-documenting.",
        ),
    ],
    &[
        Slot::Str("The architecture for "),
        Slot::Topic,
        Slot::Str(
            " emerges from recognizing which operations are fundamental and which are derivable from them. We identify the minimal set of core operations. Everything else builds composably from these foundations. This parsimonious approach yields elegant and maintainable code.",
        ),
    ],
    &[
        Slot::Str("Crafting an implementation of "),
        Slot::Topic,
        Slot::Str(
            " that proves robust means anticipating failure modes and addressing them preventatively. We identify what can go wrong and design the system to prevent those failures. Error handling is not an afterthought but integral to the design. The result is code that fails gracefully when problems arise.",
        ),
    ],
    &[
        Slot::Str("The implementation strategy for "),
        Slot::Topic,
        Slot::Str(
            " prioritizes making illegal states unrepresentable rather than detecting them after the fact. We use the type system and structure to rule out erroneous combinations. Prevention is more effective than detection. The code prevents entire categories of bugs through thoughtful design.",
        ),
    ],
    &[
        Slot::Str("Implementing "),
        Slot::Topic,
        Slot::Str(
            " successfully requires understanding the constraints and degrees of freedom in the problem space. We identify what must be true and what can vary. This understanding guides structural decisions throughout. The implementation naturally accommodates legitimate variation while preventing illegitimate choices.",
        ),
    ],
];
