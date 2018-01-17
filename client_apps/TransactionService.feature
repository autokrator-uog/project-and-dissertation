Feature: Transaction Service
    The Transaction Service is responsible for handling user requests to send money.
    The Transaction Service is responsible for allowing users to check the status of their transactions.

    Background:
      Given there is a user named Alice
      And Alice is logged in with a valid session token
      And there is a user named Bob

    Scenario: Alice requests a transaction.
      When Alice submits a transaction request to send £50 to Bob
      Then the transaction service created a PendingTransaction with id "1" and put it on the event bus
      And the transaction service has stored the PendingTransaction with id "1" in the collection of Pending Transactions.

    Scenario: Alice's browser requests the status of her transaction, and it is ready
      Given the status of Transaction ID 1 is "ACCEPTED"
      When Alice's browser submits a polling request for the status of transaction "1"
      Then the transaction service responds with "ACCEPTED"

    Scenario: Alice's browser requests the status of her transaction, and it is still being processed
      Given the status of Transaction ID 1 is "PENDING"
      When Alice's browser submits a polling request for the status of transaction "1"
      Then the transaction service responds with "PENDING"

    Scenario: Transaction is accepted
      Given there exists a PendingTransaction with id "1"
      When the transaction service receives an AcceptedTransaction event from the event bus
      Then the status of the transaction with id "1" is "ACCEPTED"
      And the transaction is removed from the collection of pending transactions

    Scenario: Transaction is rejected
      Given there exists a PendingTransaction with id "1"
      When the transaction service receives a RejectedTransaction event from the event bus
      Then the status of the transaction with id "1" is "REJECTED"
      And the transaction is removed from the collection of pending transactions
