Feature: Constructing a validated TerrainNoise

  Scenario: Constructing a TerrainNoise with valid values succeeds
    When a TerrainNoise is constructed with frequency 1.5, 4 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the TerrainNoise is constructed successfully
    And the TerrainNoise has frequency 1.5
    And the TerrainNoise has 4 octaves
    And the TerrainNoise has persistence 0.5
    And the TerrainNoise has lacunarity 2.0
    And the TerrainNoise has amplitude 0.12
    And the TerrainNoise has redistribution exponent 1.4
    And the TerrainNoise has no terrace levels

  Scenario: Constructing a TerrainNoise with terrace levels set succeeds
    When a TerrainNoise is constructed with frequency 2.5, 5 octaves, persistence 0.55, lacunarity 2.2, amplitude 0.3, redistribution exponent 2.2, and 6 terrace levels
    Then the TerrainNoise is constructed successfully
    And the TerrainNoise has 6 terrace levels

  Scenario: Constructing a TerrainNoise with a non-positive frequency fails
    When a TerrainNoise is constructed with frequency 0.0, 4 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-frequency error of 0.0

  Scenario: Constructing a TerrainNoise with 0 octaves fails
    When a TerrainNoise is constructed with frequency 1.5, 0 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-octaves error of 0

  Scenario: Constructing a TerrainNoise with more than 8 octaves fails
    When a TerrainNoise is constructed with frequency 1.5, 9 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-octaves error of 9

  Scenario: Constructing a TerrainNoise with a persistence above 1.0 fails
    When a TerrainNoise is constructed with frequency 1.5, 4 octaves, persistence 1.1, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-persistence error of 1.1

  Scenario: Constructing a TerrainNoise with a lacunarity of 1.0 or below fails
    When a TerrainNoise is constructed with frequency 1.5, 4 octaves, persistence 0.5, lacunarity 1.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-lacunarity error of 1.0

  Scenario: Constructing a TerrainNoise with a negative amplitude fails
    When a TerrainNoise is constructed with frequency 1.5, 4 octaves, persistence 0.5, lacunarity 2.0, amplitude -0.1, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-amplitude error of -0.1

  Scenario: Constructing a TerrainNoise with a non-positive redistribution exponent fails
    When a TerrainNoise is constructed with frequency 1.5, 4 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 0.0, and no terrace levels
    Then the construction fails with an invalid-redistribution-exponent error of 0.0

  Scenario: Constructing a TerrainNoise with 1 terrace level fails
    When a TerrainNoise is constructed with frequency 1.5, 4 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and 1 terrace level
    Then the construction fails with an invalid-terrace-levels error of 1

  Scenario: Constructing a TerrainNoise with a NaN frequency fails
    When a TerrainNoise is constructed with frequency NaN, 4 octaves, persistence 0.5, lacunarity 2.0, amplitude 0.12, redistribution exponent 1.4, and no terrace levels
    Then the construction fails with an invalid-frequency error of NaN
