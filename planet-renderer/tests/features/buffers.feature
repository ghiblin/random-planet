Feature: Vertex and index buffer packing

  Scenario: Packing a vertex list produces a correctly sized buffer
    Given a vertex list with 2 vertices
    When the vertex list is packed into a vertex buffer
    Then the buffer's byte length equals the vertex count times the vertex stride

  Scenario: Packing an index list produces a correctly sized buffer
    Given an index list with 3 indices
    When the index list is packed into an index buffer
    Then the buffer's byte length equals the index count times the index size

  Scenario: Packing an empty vertex list produces an empty buffer
    Given an empty vertex list
    When the vertex list is packed into a vertex buffer
    Then the buffer is empty
