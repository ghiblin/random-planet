Feature: Constructing a validated Rgb color

  Scenario: Constructing an Rgb with all channels in range succeeds
    When an Rgb is constructed with r 0.2, g 0.4, b 0.6
    Then the Rgb is constructed successfully
    And the Rgb has r 0.2, g 0.4, b 0.6

  Scenario: Constructing an Rgb with channels at the exact boundaries succeeds
    When an Rgb is constructed with r 0.0, g 1.0, b 0.0
    Then the Rgb is constructed successfully

  Scenario: Constructing an Rgb with a channel below 0.0 fails
    When an Rgb is constructed with r -0.1, g 0.5, b 0.5
    Then the construction fails with an out-of-range error of r -0.1, g 0.5, b 0.5

  Scenario: Constructing an Rgb with a channel above 1.0 fails
    When an Rgb is constructed with r 0.5, g 1.5, b 0.5
    Then the construction fails with an out-of-range error of r 0.5, g 1.5, b 0.5

  Scenario: Constructing an Rgb with a NaN channel fails
    When an Rgb is constructed with r NaN, g 0.5, b 0.5
    Then the construction fails with an out-of-range error where r is NaN
