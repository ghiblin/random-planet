Feature: Bundling validated subdivision and color parameters into PresetParams

  Scenario: Constructing PresetParams bundles all 5 fields unchanged
    Given a MinEdgeLength of 0.4, an ElevationNoiseRange of low -0.1 and high 0.1, a NormalNoiseRange of low -0.05 and high 0.05, a SplitPointVariance of 0.15, and a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white
    When a PresetParams is constructed from those 5 values
    Then the PresetParams has a MinEdgeLength of 0.4
    And the PresetParams has an ElevationNoiseRange of low -0.1 and high 0.1
    And the PresetParams has a NormalNoiseRange of low -0.05 and high 0.05
    And the PresetParams has a SplitPointVariance of 0.15
    And the PresetParams's ColorGradient samples elevation 0.0 to black

  Scenario: Two PresetParams built from identical arguments are equal
    Given a MinEdgeLength of 0.4, an ElevationNoiseRange of low -0.1 and high 0.1, a NormalNoiseRange of low -0.05 and high 0.05, a SplitPointVariance of 0.15, and a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white
    When two PresetParams are constructed from those same 5 values, separately
    Then the two PresetParams are identical
