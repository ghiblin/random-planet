Feature: Constructing a validated MinEdgeLength

  Scenario: Constructing a MinEdgeLength with a positive value succeeds
    When a MinEdgeLength is constructed with value 0.5
    Then the MinEdgeLength is constructed successfully
    And the MinEdgeLength has value 0.5

  Scenario: Constructing a MinEdgeLength with zero succeeds
    When a MinEdgeLength is constructed with value 0.0
    Then the MinEdgeLength is constructed successfully

  Scenario: Constructing a MinEdgeLength with a negative value fails
    When a MinEdgeLength is constructed with value -0.1
    Then the construction fails with a negative-value error of -0.1

  Scenario: Constructing a MinEdgeLength with NaN fails
    When a MinEdgeLength is constructed with value NaN
    Then the construction fails with a negative-value error of NaN

  Scenario: The default MinEdgeLength has value 0.1
    Given the default MinEdgeLength
    Then the MinEdgeLength has value 0.1
