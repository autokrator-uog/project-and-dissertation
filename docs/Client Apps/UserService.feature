Feature: User Service
    The User service is responsible for user registration and log in.
    The User service is used by other micro-services as a single source of truth to validate session tokens.

    Scenario: Account creation
      When Alice signs up to create an account
      Then a CreateAccount event is added to the event bus
      And Alice is given her account ID

    Scenario: Log in
      Given there is a user named Alice
      And Alice enters the correct username / password
      When Alice requests to log in
      Then Alice is returned a session token in a cookie

    Scenario: Log in with invalid password
      Given there is a user named Alice
      And Alice enters an incorrect username / password
      When Alice requests to log in
      Then Alice is returned a 401 Unauthorized

    Scenario: A Micro-service wants to validate a session token (and it's valid)
      Given there is a valid session token "ASDF"
      And the session token has expiry in "332" seconds
      When the user service receives a token validation request
      Then the micro service receives a "valid" response with expiry "332"

    Scenario: A Micro-service wants to validate a session token (and it's invalid)
      Given there is does not exist a session token "ASDF"
      When the user service receives a token validation request
      Then the micro service receives an "invalid" response
