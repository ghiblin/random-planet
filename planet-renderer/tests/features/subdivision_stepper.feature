Feature: Stepping through subdivision one round at a time

  Scenario: Stepping advances the mesh by exactly one subdivision round
    Given a SubdivisionStepper constructed from the icosahedron mesh with max depth 3
    When the stepper is stepped once using the uniform red-split strategy
    Then the step succeeds
    And the stepper has completed 1 rounds
    And the stepper's mesh has 80 triangles

  Scenario: Stepping repeatedly stops advancing once max depth is reached
    Given a SubdivisionStepper constructed from the icosahedron mesh with max depth 1
    When the stepper is stepped once using the uniform red-split strategy
    And the stepper is stepped again using the uniform red-split strategy
    Then the second step does not succeed
    And the stepper has completed 1 rounds
    And the stepper's mesh has 80 triangles
