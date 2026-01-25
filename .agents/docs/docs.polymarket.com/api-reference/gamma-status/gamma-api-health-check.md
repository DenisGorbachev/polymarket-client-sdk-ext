# Gamma API Health check

## OpenAPI

````yaml api-reference/gamma-openapi.json get /status
openapi: 3.0.3
info:
  title: Markets API
  version: 1.0.0
  description: REST API specification for public endpoints used by the Markets service.
servers:
  - url: https://gamma-api.polymarket.com
    description: Polymarket Gamma API Production Server
security: []
tags:
  - name: Gamma Status
    description: Gamma API status and health check
  - name: Sports
    description: Sports-related endpoints including teams and game data
  - name: Tags
    description: Tag management and related tag operations
  - name: Events
    description: Event management and event-related operations
  - name: Markets
    description: Market data and market-related operations
  - name: Comments
    description: Comment system and user interactions
  - name: Series
    description: Series management and related operations
  - name: Profiles
    description: User profile management
  - name: Search
    description: Search functionality across different entity types
paths:
  /status:
    get:
      tags:
        - Gamma Status
      summary: Gamma API Health check
      operationId: getGammaStatus
      responses:
        '200':
          description: OK
          content:
            text/plain:
              schema:
                type: string
                example: OK

````
