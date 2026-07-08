Feature: Constructing a Seed

  Scenario: Constructing a Seed from a u64 value
    When a Seed is constructed with value 42
    Then the Seed has value 42

  Scenario: The default Seed value is 0
    Given the default Seed
    Then the Seed has value 0
