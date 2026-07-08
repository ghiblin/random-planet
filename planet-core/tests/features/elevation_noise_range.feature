Feature: Constructing a validated ElevationNoiseRange

  Scenario: Constructing an ElevationNoiseRange with low less than high succeeds
    When an ElevationNoiseRange is constructed with low -0.1 and high 0.2
    Then the ElevationNoiseRange is constructed successfully
    And the ElevationNoiseRange has low -0.1
    And the ElevationNoiseRange has high 0.2

  Scenario: Constructing an ElevationNoiseRange with equal low and high succeeds
    When an ElevationNoiseRange is constructed with low 0.0 and high 0.0
    Then the ElevationNoiseRange is constructed successfully

  Scenario: Constructing an ElevationNoiseRange with low greater than high fails
    When an ElevationNoiseRange is constructed with low 0.5 and high 0.1
    Then the construction fails with an invalid-range error of low 0.5 and high 0.1

  Scenario: The default ElevationNoiseRange has low -0.05 and high 0.05
    Given the default ElevationNoiseRange
    Then the ElevationNoiseRange has low -0.05
    And the ElevationNoiseRange has high 0.05
