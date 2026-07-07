Feature: Vertex and index buffer packing

  Scenario: Packing the cube's vertex list produces a correctly sized buffer
    Given the cube's fixed vertex list
    When the vertex list is packed into a vertex buffer
    Then the buffer's byte length equals the vertex count times the vertex stride

  Scenario: Packing the cube's index list produces a correctly sized buffer
    Given the cube's fixed index list
    When the index list is packed into an index buffer
    Then the buffer's byte length equals the index count times the index size

  Scenario: Packing an empty vertex list produces an empty buffer
    Given an empty vertex list
    When the vertex list is packed into a vertex buffer
    Then the buffer is empty
