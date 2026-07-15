Feature: Constructing SubdivisionArgs with defaults

  Scenario: Constructing SubdivisionArgs with explicit steps, mode, and seed
    Given Steps constructed with 5
    When SubdivisionArgs is constructed with those steps, the UniformRedSplit mode, and seed 7
    Then the SubdivisionArgs has 5 steps
    And the SubdivisionArgs has mode SubdivisionMode::UniformRedSplit
    And the SubdivisionArgs has seed 7

  Scenario: Omitting steps defaults to 3
    When SubdivisionArgs is constructed with no steps, the UniformRedSplit mode, and seed 7
    Then the SubdivisionArgs has 3 steps

  Scenario: Omitting mode defaults to the default UniformRedSplit mode
    Given Steps constructed with 2
    When SubdivisionArgs is constructed with those steps, no mode, and seed 7
    Then the SubdivisionArgs has mode SubdivisionMode::UniformRedSplit

  Scenario: Omitting seed defaults to seed 0
    Given Steps constructed with 2
    When SubdivisionArgs is constructed with those steps, the UniformRedSplit mode, and no seed
    Then the SubdivisionArgs has seed 0
