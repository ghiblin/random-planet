Feature: Converting a millisecond timestamp into a Seed

  Scenario: Converting a typical timestamp produces the expected Seed
    When the timestamp 1752400000000.0 is converted to a Seed
    Then the resulting Seed has value 1752400000000

  Scenario: Converting a negative timestamp saturates to zero
    When the timestamp -1.0 is converted to a Seed
    Then the resulting Seed has value 0

  Scenario: Converting NaN saturates to zero
    When the timestamp NaN is converted to a Seed
    Then the resulting Seed has value 0

  Scenario: Converting a timestamp beyond u64's range saturates to the maximum
    When the timestamp 1e30 is converted to a Seed
    Then the resulting Seed has the maximum u64 value

  Scenario: Converting the same timestamp twice produces equal Seeds
    When the timestamp 1752400000000.0 is converted to a Seed twice
    Then both resulting Seeds are equal
