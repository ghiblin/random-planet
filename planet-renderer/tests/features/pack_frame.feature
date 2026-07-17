Feature: Packing a Mesh and its colors into a single PackedFrame

  Scenario: Packing a cube Mesh bundles all four render buffers
    Given a Mesh constructed by Mesh::cube with side 1.0
    And normals finalized for that mesh
    And a distinct Rgb color for each of the mesh's vertices
    When the mesh and colors are packed into a PackedFrame
    Then the PackedFrame's vertex_bytes_smooth equals packing the mesh's smooth-shaded render vertices
    And the PackedFrame's vertex_bytes_flat equals packing the mesh's flat-shaded render vertices
    And the PackedFrame's index_bytes equals packing the mesh's render indices
    And the PackedFrame's line_index_bytes equals packing the mesh's render line indices

  Scenario: Packing an empty Mesh produces four empty buffers
    Given an empty Mesh with no vertices and no triangles
    When the mesh and colors are packed into a PackedFrame
    Then the PackedFrame's vertex_bytes_smooth is empty
    And the PackedFrame's index_bytes is empty
