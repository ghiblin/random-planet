Feature: Uniform buffer packing

  Scenario: Packing a view-projection matrix produces a correctly sized uniform buffer
    Given a view-projection matrix computed from a Camera
    When the matrix is packed into a uniform buffer
    Then the buffer's byte length equals 64 bytes
