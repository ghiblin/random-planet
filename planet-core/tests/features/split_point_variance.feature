Feature: Constructing a validated SplitPointVariance

  Scenario: Constructing a SplitPointVariance with a positive value succeeds
    When a SplitPointVariance is constructed with value 0.3
    Then the SplitPointVariance is constructed successfully
    And the SplitPointVariance has value 0.3

  Scenario: Constructing a SplitPointVariance with zero succeeds
    When a SplitPointVariance is constructed with value 0.0
    Then the SplitPointVariance is constructed successfully

  Scenario: Constructing a SplitPointVariance with a negative value fails
    When a SplitPointVariance is constructed with value -0.1
    Then the construction fails with a negative-value error of -0.1

  Scenario: Constructing a SplitPointVariance with NaN fails
    When a SplitPointVariance is constructed with value NaN
    Then the construction fails with a negative-value error of NaN

  Scenario: The default SplitPointVariance has value 0.1
    Given the default SplitPointVariance
    Then the SplitPointVariance has value 0.1
