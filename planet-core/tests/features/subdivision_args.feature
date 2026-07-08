Feature: Constructing SubdivisionArgs with defaults

  Scenario: Constructing SubdivisionArgs with explicit steps and mode
    Given Steps constructed with 5
    When SubdivisionArgs is constructed with those steps and the UniformRedSplit mode
    Then the SubdivisionArgs has 5 steps
    And the SubdivisionArgs has the UniformRedSplit mode

  Scenario: Omitting steps defaults to 3
    When SubdivisionArgs is constructed with no steps and the UniformRedSplit mode
    Then the SubdivisionArgs has 3 steps

  Scenario: Omitting mode defaults to UniformRedSplit
    Given Steps constructed with 2
    When SubdivisionArgs is constructed with those steps and no mode
    Then the SubdivisionArgs has the UniformRedSplit mode
