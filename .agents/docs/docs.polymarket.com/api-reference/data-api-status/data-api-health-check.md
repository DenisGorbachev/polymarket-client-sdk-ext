# Data API Health check

## OpenAPI

````yaml api-reference/data-api-openapi.yaml get /
openapi: 3.0.3
info:
  title: Polymarket Data API
  version: 1.0.0
  description: >
    HTTP API for Polymarket data. This specification documents all public
    routes.
servers:
  - url: https://data-api.polymarket.com
    description: Relative server (same host)
security: []
tags:
  - name: Data API Status
    description: Data API health check
  - name: Core
  - name: Builders
  - name: Misc
paths:
  /:
    get:
      tags:
        - Data API Status
      summary: Data API Health check
      operationId: getDataApiHealth
      responses:
        '200':
          description: OK
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/HealthResponse'
components:
  schemas:
    HealthResponse:
      type: object
      properties:
        data:
          type: string
          example: OK

````
