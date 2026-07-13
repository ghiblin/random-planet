Feature: Selecting a Preset's parameters

  Scenario: The Earthy preset has its configured parameters
    When Preset::Earthy's params are requested
    Then the PresetParams has a MinEdgeLength of 0.35
    And the PresetParams has an ElevationNoiseRange of low -0.05 and high 0.15
    And the PresetParams has a NormalNoiseRange of low -0.05 and high 0.05
    And the PresetParams has a SplitPointVariance of 0.1
    And the PresetParams has an OceanQuota of 0.4

  Scenario: The Volcano preset has its configured parameters
    When Preset::Volcano's params are requested
    Then the PresetParams has a MinEdgeLength of 0.25
    And the PresetParams has an ElevationNoiseRange of low -0.05 and high 0.35
    And the PresetParams has a NormalNoiseRange of low -0.1 and high 0.1
    And the PresetParams has a SplitPointVariance of 0.2
    And the PresetParams has no OceanQuota

  Scenario: The Rocky preset has its configured parameters
    When Preset::Rocky's params are requested
    Then the PresetParams has a MinEdgeLength of 0.3
    And the PresetParams has an ElevationNoiseRange of low -0.2 and high 0.2
    And the PresetParams has a NormalNoiseRange of low -0.15 and high 0.15
    And the PresetParams has a SplitPointVariance of 0.25
    And the PresetParams has no OceanQuota

  Scenario: Earthy's color gradient samples its own lowest and highest configured elevations to its first and last stops' colors
    When Preset::Earthy's params are requested
    Then sampling its color gradient at elevation 0.85 returns Rgb r 0.05, g 0.15, b 0.45
    And sampling its color gradient at elevation 1.15 returns Rgb r 0.95, g 0.95, b 0.95

  Scenario: Volcano's color gradient samples its own lowest and highest configured elevations to its first and last stops' colors
    When Preset::Volcano's params are requested
    Then sampling its color gradient at elevation 0.95 returns Rgb r 0.1, g 0.05, b 0.05
    And sampling its color gradient at elevation 1.35 returns Rgb r 1.0, g 0.85, b 0.3

  Scenario: Rocky's color gradient samples its own lowest and highest configured elevations to its first and last stops' colors
    When Preset::Rocky's params are requested
    Then sampling its color gradient at elevation 0.8 returns Rgb r 0.3, g 0.28, b 0.26
    And sampling its color gradient at elevation 1.2 returns Rgb r 0.8, g 0.78, b 0.74

  Scenario: Preset::params is deterministic
    When Preset::Rocky's params are requested twice
    Then both PresetParams are identical

  Scenario: The default Preset is Earthy
    Given the default Preset
    Then the Preset equals Preset::Earthy

  Scenario: Preset::ALL lists all three presets in a fixed order
    When Preset::ALL is requested
    Then Preset::ALL equals Earthy, Volcano, Rocky in that order

  Scenario: Every Preset has a human-readable name
    When each Preset's name is requested
    Then Preset::Earthy's name is "Earthy"
    And Preset::Volcano's name is "Volcano"
    And Preset::Rocky's name is "Rocky"

  Scenario: Every Preset has a non-empty description
    When each Preset's description is requested
    Then Preset::Earthy's description is non-empty
    And Preset::Volcano's description is non-empty
    And Preset::Rocky's description is non-empty
