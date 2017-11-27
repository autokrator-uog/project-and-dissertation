# Client Apps

![Diagram](./ClientSide.png)

Above is a diagram of client apps architecture and how the main communicaiton will occur within them.


## Event Schemas

### AccountCreation

```
{
  "AccountID": Integer,
  "PasswordHash": String,  // used on redelivery to build up internal DB state
  "Timestamp": String
}
```

### PendingTransaction

```
{
  "TransactionID": Integer,
  "FromAccountID": Integer,
  "ToAccountID": Integer,
  "Amount": Decimal
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
    "Amount": Decimal,
    "Timestamp": String
}
```
