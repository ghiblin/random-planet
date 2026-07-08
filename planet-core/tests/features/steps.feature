Feature: Constructing validated Steps

  Scenario: Constructing Steps within the allowed range succeeds
    When Steps is constructed with 5
    Then the Steps is constructed successfully
    And the Steps has value 5

  Scenario: The maximum allowed step count is accepted
    When Steps is constructed with 8
    Then the Steps is constructed successfully
    And the Steps has value 8

  Scenario: Requesting more steps than the maximum fails
    When Steps is constructed with 9
    Then the construction fails with an exceeds-maximum error of 9 steps and max 8

  Scenario: The default Steps value is 3
    Given the default Steps
    Then the Steps has value 3
