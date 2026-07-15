Feature: Bundling validated terrain-noise, color, and subdivision parameters into PresetParams

  Scenario: Constructing PresetParams bundles all 4 fields unchanged
    Given a TerrainNoise with amplitude 0.12, a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white, an OceanQuota of 0.2, and SubdivisionMode::UniformRedSplit
    When a PresetParams is constructed from those 4 values
    Then the PresetParams has a TerrainNoise with amplitude 0.12
    And the PresetParams's ColorGradient samples elevation 0.0 to black
    And the PresetParams has an OceanQuota of 0.2
    And the PresetParams has subdivision mode SubdivisionMode::UniformRedSplit

  Scenario: Two PresetParams built from identical arguments are equal
    Given a TerrainNoise with amplitude 0.12, a ColorGradient with stops at elevation 0.0 color black and elevation 1.0 color white, an OceanQuota of 0.2, and SubdivisionMode::UniformRedSplit
    When two PresetParams are constructed from those same 4 values, separately
    Then the two PresetParams are identical
