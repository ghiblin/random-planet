Feature: Converting a Mesh into render vertices

  Scenario: Converting a cube Mesh into render vertices produces one vertex per face corner
    Given a Mesh constructed by Mesh::cube with side 1.0
    And normals finalized for that mesh
    When the mesh is converted into render vertices
    Then the render vertex list has 36 vertices
    And each render vertex's normal equals its source vertex's normal

  Scenario: Converting an empty Mesh into render vertices produces an empty list
    Given an empty Mesh with no vertices and no triangles
    When the mesh is converted into render vertices
    Then the render vertex list is empty

  Scenario: Converting a Mesh into render vertices assigns each render vertex the color of its source vertex
    Given a Mesh constructed by Mesh::cube with side 1.0
    And normals finalized for that mesh
    And a distinct Rgb color for each of the mesh's vertices
    When the mesh is converted into render vertices using those colors
    Then each render vertex's color equals its source vertex's Rgb
