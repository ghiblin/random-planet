Feature: Sampling a ColorGradient

  Scenario: Constructing a ColorGradient with at least 2 strictly ascending stops succeeds
    When a ColorGradient is constructed with stops at elevation 0.0 color black and elevation 1.0 color white
    Then the ColorGradient is constructed successfully

  Scenario: Constructing a ColorGradient with fewer than 2 stops fails
    When a ColorGradient is constructed with a single stop at elevation 0.0 color black
    Then the construction fails with a too-few-stops error of count 1

  Scenario: Constructing a ColorGradient with non-ascending stops fails
    When a ColorGradient is constructed with stops at elevation 1.0 color white and elevation 0.0 color black
    Then the construction fails with a stops-not-strictly-ascending error at index 1

  Scenario: Constructing a ColorGradient with two stops at the same elevation fails
    When a ColorGradient is constructed with stops at elevation 0.5 color black and elevation 0.5 color white
    Then the construction fails with a stops-not-strictly-ascending error at index 1

  Scenario: Sampling below the first stop clamps to the first stop's color
    Given a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white
    When the ColorGradient is sampled at elevation -5.0
    Then the sampled Rgb equals black

  Scenario: Sampling above the last stop clamps to the last stop's color
    Given a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white
    When the ColorGradient is sampled at elevation 5.0
    Then the sampled Rgb equals white

  Scenario: Sampling exactly at a stop's elevation returns that stop's color exactly
    Given a ColorGradient with stops at elevation 0.0 color black, elevation 0.5 color gray, and elevation 1.0 color white
    When the ColorGradient is sampled at elevation 0.5
    Then the sampled Rgb equals gray

  Scenario: Sampling exactly at an interior stop's elevation returns that stop's color exactly, even when neither bracketing stop is black
    Given a ColorGradient with stops at elevation 0.0 color with r 0.12, g 0.34, b 0.56, elevation 0.5 color with r 0.65, g 0.43, b 0.21, and elevation 1.0 color with r 0.91, g 0.82, b 0.73
    When the ColorGradient is sampled at elevation 0.5
    Then the sampled Rgb has r 0.65, g 0.43, b 0.21

  Scenario: Sampling halfway between two stops linearly interpolates each channel
    Given a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white
    When the ColorGradient is sampled at elevation 0.5
    Then the sampled Rgb has r 0.5, g 0.5, b 0.5
