Feature: Constructing a validated NormalNoiseRange

  Scenario: Constructing a NormalNoiseRange with low less than high succeeds
    When a NormalNoiseRange is constructed with low -0.1 and high 0.2
    Then the NormalNoiseRange is constructed successfully
    And the NormalNoiseRange has low -0.1
    And the NormalNoiseRange has high 0.2

  Scenario: Constructing a NormalNoiseRange with equal low and high succeeds
    When a NormalNoiseRange is constructed with low 0.0 and high 0.0
    Then the NormalNoiseRange is constructed successfully

  Scenario: Constructing a NormalNoiseRange with low greater than high fails
    When a NormalNoiseRange is constructed with low 0.5 and high 0.1
    Then the construction fails with an invalid-range error of low 0.5 and high 0.1

  Scenario: The default NormalNoiseRange has low -0.05 and high 0.05
    Given the default NormalNoiseRange
    Then the NormalNoiseRange has low -0.05
    And the NormalNoiseRange has high 0.05
