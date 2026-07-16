Feature: Converting a Mesh into render vertices

  Scenario: Converting a cube Mesh into render vertices produces one vertex per face corner
    Given a Mesh constructed by Mesh::cube with side 1.0
    And normals finalized for that mesh
    When the mesh is converted into render vertices
    Then the render vertex list has 36 vertices

  Scenario: Converting a Mesh into render vertices with smooth shading assigns each render vertex its source vertex's normal
    Given a Mesh constructed by Mesh::cube with side 1.0
    And normals finalized for that mesh
    When the mesh is converted into render vertices with smooth shading
    Then each render vertex's normal equals its source vertex's normal

  Scenario: Converting a Mesh into render vertices with flat shading assigns every corner of a face that face's own normal
    Given a Mesh constructed by Mesh::cube with side 1.0
    And normals finalized for that mesh
    When the mesh is converted into render vertices with flat shading
    Then every render vertex belonging to the same face has that face's normal

  Scenario: Converting a Mesh with a degenerate face into render vertices with flat shading falls back to a zero normal without panicking
    Given a Mesh with 3 vertices at the same position
    And a triangle index-triple (0, 1, 2)
    And normals finalized for that mesh
    When the mesh is converted into render vertices with flat shading
    Then every render vertex belonging to that face has normal (0.0, 0.0, 0.0)

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
