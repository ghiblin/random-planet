Feature: Selecting a Preset's parameters

  Scenario: The Earthy preset has its configured parameters
    When Preset::Earthy's params are requested
    Then the PresetParams has a MinEdgeLength of 0.35
    And the PresetParams has an ElevationNoiseRange of low -0.05 and high 0.15
    And the PresetParams has a NormalNoiseRange of low -0.05 and high 0.05
    And the PresetParams has a SplitPointVariance of 0.1

  Scenario: The Volcano preset has its configured parameters
    When Preset::Volcano's params are requested
    Then the PresetParams has a MinEdgeLength of 0.25
    And the PresetParams has an ElevationNoiseRange of low -0.05 and high 0.35
    And the PresetParams has a NormalNoiseRange of low -0.1 and high 0.1
    And the PresetParams has a SplitPointVariance of 0.2

  Scenario: The Rocky preset has its configured parameters
    When Preset::Rocky's params are requested
    Then the PresetParams has a MinEdgeLength of 0.3
    And the PresetParams has an ElevationNoiseRange of low -0.2 and high 0.2
    And the PresetParams has a NormalNoiseRange of low -0.15 and high 0.15
    And the PresetParams has a SplitPointVariance of 0.25

  Scenario: Each preset's color gradient samples its own lowest configured elevation to its first stop's color
    When Preset::Earthy's params are requested
    Then sampling its color gradient at elevation 0.85 returns Rgb r 0.05, g 0.15, b 0.45

  Scenario: Each preset's color gradient samples its own highest configured elevation to its last stop's color
    When Preset::Volcano's params are requested
    Then sampling its color gradient at elevation 1.35 returns Rgb r 1.0, g 0.85, b 0.3

  Scenario: Preset::params is deterministic
    When Preset::Rocky's params are requested twice
    Then both PresetParams are identical

  Scenario: The default Preset is Earthy
    Given the default Preset
    Then the Preset equals Preset::Earthy
