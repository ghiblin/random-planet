Feature: Pacing the subdivision growth-animation frame reveal

  Scenario: Ticking after the pacing interval has elapsed advances to the next frame
    Given a GrowthAnimation constructed with 3 frames and started at 0.0ms
    When the GrowthAnimation is ticked at 150.0ms
    Then the tick returns true
    And the GrowthAnimation's current frame index is 1

  Scenario: Ticking before the pacing interval has elapsed does not advance
    Given a GrowthAnimation constructed with 3 frames and started at 0.0ms
    When the GrowthAnimation is ticked at 50.0ms
    Then the tick returns false
    And the GrowthAnimation's current frame index is 0

  Scenario: Ticking a single-frame GrowthAnimation never advances
    Given a GrowthAnimation constructed with 1 frame and started at 0.0ms
    When the GrowthAnimation is ticked at 1000.0ms
    Then the tick returns false
    And the GrowthAnimation's current frame index is 0

  Scenario: Ticking at the last frame never advances past the end
    Given a GrowthAnimation constructed with 2 frames and started at 0.0ms
    And the GrowthAnimation has already been ticked at 150.0ms
    When the GrowthAnimation is ticked at 300.0ms
    Then the tick returns false
    And the GrowthAnimation's current frame index is 1
