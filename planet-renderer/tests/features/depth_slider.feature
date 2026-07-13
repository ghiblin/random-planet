Feature: Parsing a depth-slider DOM value into validated Steps

  Scenario: Parsing a value within range succeeds
    When the depth-slider value "5" is parsed
    Then the parsed Steps has value 5

  Scenario: Parsing the minimum boundary value succeeds
    When the depth-slider value "0" is parsed
    Then the parsed Steps has value 0

  Scenario: Parsing the maximum boundary value succeeds
    When the depth-slider value "8" is parsed
    Then the parsed Steps has value 8

  Scenario: Parsing a value above the maximum fails
    When the depth-slider value "9" is parsed
    Then the parsing fails with an invalid-steps error

  Scenario: Parsing a non-numeric value fails
    When the depth-slider value "abc" is parsed
    Then the parsing fails with a not-a-number error
