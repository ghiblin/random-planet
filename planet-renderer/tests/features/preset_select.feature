Feature: Parsing a preset-select DOM value into a Preset

  Scenario: Parsing a recognized preset name returns the matching Preset
    When the preset-select value "Volcano" is parsed
    Then the parsed Preset is Volcano

  Scenario: Every Preset::ALL member's own name round-trips to itself
    When each of Preset::ALL's names is parsed
    Then every parsed Preset equals its source Preset

  Scenario: Parsing an unrecognized value returns no Preset
    When the preset-select value "Unknown" is parsed
    Then no Preset is returned
