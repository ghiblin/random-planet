Feature: Constructing a validated OceanQuota

  Scenario: Constructing an OceanQuota with a value within 0.0 and 1.0 succeeds
    When an OceanQuota is constructed with value 0.4
    Then the OceanQuota is constructed successfully
    And the OceanQuota has value 0.4

  Scenario: Constructing an OceanQuota with the boundary value 0.0 succeeds
    When an OceanQuota is constructed with value 0.0
    Then the OceanQuota is constructed successfully
    And the OceanQuota has value 0.0

  Scenario: Constructing an OceanQuota with the boundary value 1.0 succeeds
    When an OceanQuota is constructed with value 1.0
    Then the OceanQuota is constructed successfully
    And the OceanQuota has value 1.0

  Scenario: Constructing an OceanQuota with a negative value fails
    When an OceanQuota is constructed with value -0.1
    Then the construction fails with an out-of-range error of -0.1

  Scenario: Constructing an OceanQuota with a value above 1.0 fails
    When an OceanQuota is constructed with value 1.5
    Then the construction fails with an out-of-range error of 1.5

  Scenario: Constructing an OceanQuota with NaN fails
    When an OceanQuota is constructed with value NaN
    Then the construction fails with an out-of-range error of NaN

  Scenario: The default OceanQuota has value 0.3
    Given the default OceanQuota
    Then the OceanQuota has value 0.3
