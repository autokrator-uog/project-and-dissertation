# Client Apps

![Diagram](./ClientSide_5_12_17.png)

Above is a diagram of client apps architecture and how the main communicaiton will occur within them.


## Event Schemas

### AccountCreationRequest

This event has no real need for a body, because the Accounts service doesn't need to care about users, it just gets an instruction to create an account. To track which account request it is, add the option for the user service to specify a request ID.
```
{
  "RequestID": String,
  "Timestamp": String
}
```

### AccountCreated

```
{
  "RequestID": String,
  "AccountID": Integer
}
```

### PendingTransaction

```
{
  "TransactionID": Integer,
  "FromAccountID": Integer,
  "ToAccountID": Integer,
  "Amount": String // string of decimal representation (2 decimal places)
}
```

### AcceptedTransaction / RejectedTransaction
```
{
  "TransactionID": Integer,
}
```

### ConfirmedCredit / ConfimredDebit
```
{
    "AccountID": Integer,
    "Amount": String, // string of decimal representation (2 decimal places)
    "Timestamp": String
}
```
