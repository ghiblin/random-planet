Feature: Streaming the subdivision growth-animation frame reveal

  Scenario: The first pushed frame is revealed immediately, with no pacing delay
    Given a new GrowthAnimation with no frames yet
    When a frame is pushed at 0.0ms
    Then the GrowthAnimation's current frame is that frame

  Scenario: A second frame pushed before the pacing interval has elapsed is not yet revealed
    Given a new GrowthAnimation with no frames yet
    And a frame is pushed at 0.0ms
    When a second, distinct frame is pushed at 50.0ms
    Then the GrowthAnimation's current frame is still the first frame

  Scenario: Ticking after the pacing interval has elapsed reveals the next pending frame
    Given a new GrowthAnimation with no frames yet
    And a frame is pushed at 0.0ms
    And a second, distinct frame is pushed at 50.0ms
    When the GrowthAnimation is ticked at 150.0ms
    Then the tick returns true
    And the GrowthAnimation's current frame is the second frame

  Scenario: Ticking with no pending frame never advances
    Given a new GrowthAnimation with no frames yet
    And a frame is pushed at 0.0ms
    When the GrowthAnimation is ticked at 1000.0ms
    Then the tick returns false
    And the GrowthAnimation's current frame is still the first frame

  Scenario: Ticking a brand-new GrowthAnimation with no frames pushed yet never advances
    Given a new GrowthAnimation with no frames yet
    When the GrowthAnimation is ticked at 1000.0ms
    Then the tick returns false
    And the GrowthAnimation has no current frame
