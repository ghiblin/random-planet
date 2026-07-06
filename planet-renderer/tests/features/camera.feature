Feature: Camera orbit and zoom

  Scenario: Orbiting updates yaw and pitch
    Given a Camera constructed with default orbit parameters
    When the camera is orbited by a mouse delta of (0.2, 0.1)
    Then the camera's yaw increases by 0.2
    And the camera's pitch increases by 0.1

  Scenario: Orbiting clamps pitch to avoid gimbal-lock flip
    Given a Camera constructed with default orbit parameters
    When the camera is orbited upward past the maximum pitch
    Then the camera's pitch stays at the maximum allowed pitch

  Scenario: Zooming in decreases distance
    Given a Camera constructed with default orbit parameters
    When the camera is zoomed in by a scroll delta of 1.0
    Then the camera's distance decreases
    And the camera's distance stays at or above the minimum distance

  Scenario: Zooming in past the minimum distance clamps
    Given a Camera constructed at the minimum distance
    When the camera is zoomed in by a scroll delta of 100.0
    Then the camera's distance stays at the minimum distance

  Scenario: Zooming out past the maximum distance clamps
    Given a Camera constructed at the maximum distance
    When the camera is zoomed out by a scroll delta of 100.0
    Then the camera's distance stays at the maximum distance
