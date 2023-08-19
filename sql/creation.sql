CREATE TABLE Accounts (
  account_id            SERIAL      PRIMARY KEY,
  current_balance       BYTEA       NOT NULL,
  name                  BYTEA       NOT NULL
  );
  
CREATE TABLE Categories (
  category_id           SERIAL      PRIMARY KEY,
  name                  BYTEA       NOT NULL,
  type                  BIT         NOT NULL
  );
  
CREATE TABLE Transactions (
  transaction_id        SERIAL      PRIMARY KEY,
  account_id            SERIAL      REFERENCES Accounts(account_id),
  category_id           SERIAL      REFERENCES Categories(category_id),
  amount                BYTEA       NOT NULL
  )