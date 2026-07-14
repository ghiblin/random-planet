Feature: Bundling validated terrain-noise and color parameters into PresetParams

  Scenario: Constructing PresetParams bundles all 3 fields unchanged
    Given a TerrainNoise with amplitude 0.12, a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white, and an OceanQuota of 0.2
    When a PresetParams is constructed from those 3 values
    Then the PresetParams has a TerrainNoise with amplitude 0.12
    And the PresetParams's ColorGradient samples elevation 0.0 to black
    And the PresetParams has an OceanQuota of 0.2

  Scenario: Two PresetParams built from identical arguments are equal
    Given a TerrainNoise with amplitude 0.12, a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white, and an OceanQuota of 0.2
    When two PresetParams are constructed from those same 3 values, separately
    Then the two PresetParams are identical
