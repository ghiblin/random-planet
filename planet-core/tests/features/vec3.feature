Feature: Vec3 basic math operations

  Scenario: Adding two vectors sums their components
    Given a Vec3 of (1.0, 2.0, 3.0)
    And a second Vec3 of (4.0, 5.0, 6.0)
    When the two vectors are added
    Then the resulting Vec3 is (5.0, 7.0, 9.0)

  Scenario: Subtracting two vectors
    Given a Vec3 of (5.0, 7.0, 9.0)
    And a second Vec3 of (4.0, 5.0, 6.0)
    When the second vector is subtracted from the first
    Then the resulting Vec3 is (1.0, 2.0, 3.0)

  Scenario: Scaling a vector by a scalar
    Given a Vec3 of (1.0, 2.0, 3.0)
    When the vector is scaled by 2.0
    Then the resulting Vec3 is (2.0, 4.0, 6.0)

  Scenario: Dot product of two orthogonal vectors is zero
    Given a Vec3 of (1.0, 0.0, 0.0)
    And a second Vec3 of (0.0, 1.0, 0.0)
    When the dot product of the two vectors is computed
    Then the result is 0.0

  Scenario: Cross product of two orthogonal unit vectors
    Given a Vec3 of (1.0, 0.0, 0.0)
    And a second Vec3 of (0.0, 1.0, 0.0)
    When the cross product of the two vectors is computed
    Then the resulting Vec3 is (0.0, 0.0, 1.0)

  Scenario: Length of a vector
    Given a Vec3 of (3.0, 4.0, 0.0)
    When the vector's length is computed
    Then the result is 5.0

  Scenario: Normalizing a non-zero vector produces a unit vector
    Given a Vec3 of (3.0, 4.0, 0.0)
    When the vector is normalized
    Then the resulting Vec3 has a length of 1.0

  Scenario: Normalizing a zero-length vector returns nothing
    Given a Vec3 of (0.0, 0.0, 0.0)
    When the vector is normalized
    Then normalization returns nothing
