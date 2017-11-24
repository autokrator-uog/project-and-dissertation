Feature: Accounts Service
    The Accounts Service is responsible for determining the balance of each account in the system.
    The Accounts Service is also responsible for validating PendingTransaction events
    and executing the necessary state changes for the transaction to occur.

    Background:
      Given there is a user named Alice
      And there is a user named Bob

    Scenario: A transaction is requested from Alice to Bob, and Alice has enough funds to complete it.
      Given Alice has a balance of £50 in her account
      And Bob has a balance of £0 in his account
      When the accounts service receives a PendingTransaction from Alice to Bob of amount £30
      Then Alice now has a balance of £20 in her account
      And Bob now has a balance of £30 in his account
      And the accounts service created a ConfirmedCredit event for Bob with amount £30
      And the accounts service created a ConfimredDebit event for Alice with amount £30

    Scenario: A transaction is requested from Alice to Bob, but Alice does not have enough funds to complete it.
      Given Alice has a balance of £50 in her account
      And Bob has a balance of £0 in his account
      When the accounts service receives a PendingTransaction from Alice to Bob of amount £51
      Then Alice still has a balance of £50 in her account
      And Bob still has a balance of £0 in his account
      And the accounts service created a RejectedTransaction event referencing the PendingTransaction

    Scenario: A user wants to view their balance
      Given Alice has a balance of £50 in her account
      And Alice is logged in with a valid session token
      When Alice requests to view her balance
      Then the Accounts Service returns £50

    Scenario: A user wants to view their balance, but has a bad session token.
      Given Alice has a balance of £50 in her account
      And Alice has an invalid session token
      When Alice requests to view her balance
      Then the Accounts Service returns a 401 Unauthorized
