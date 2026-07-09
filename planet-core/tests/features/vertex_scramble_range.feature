Feature: Constructing a validated VertexScrambleRange

  Scenario: Constructing a VertexScrambleRange with low less than high succeeds
    When a VertexScrambleRange is constructed with low -0.1 and high 0.2
    Then the VertexScrambleRange is constructed successfully
    And the VertexScrambleRange has low -0.1
    And the VertexScrambleRange has high 0.2

  Scenario: Constructing a VertexScrambleRange with equal low and high succeeds
    When a VertexScrambleRange is constructed with low 0.0 and high 0.0
    Then the VertexScrambleRange is constructed successfully

  Scenario: Constructing a VertexScrambleRange with low greater than high fails
    When a VertexScrambleRange is constructed with low 0.5 and high 0.1
    Then the construction fails with an invalid-range error of low 0.5 and high 0.1

  Scenario: Constructing a VertexScrambleRange with low at exactly -1.0 fails
    When a VertexScrambleRange is constructed with low -1.0 and high 0.0
    Then the construction fails with a low-at-or-below-negative-one error of low -1.0

  Scenario: Constructing a VertexScrambleRange with low just above -1.0 succeeds
    When a VertexScrambleRange is constructed with low -0.999 and high 0.0
    Then the VertexScrambleRange is constructed successfully

  Scenario: The default VertexScrambleRange has low -0.05 and high 0.05
    Given the default VertexScrambleRange
    Then the VertexScrambleRange has low -0.05
    And the VertexScrambleRange has high 0.05
