# `libbdgt` database schema

On the picture below logical ER-diagram of `libbdgt`'s database is shown.

![Logical ER-diagram](../pictures/er-logical.drawio.png)

DB consists of 4 tables:
- Accounts. This table contains information about user's bank accounts: 
  current balance and human-readable name (e.g. account number or 
  user-defined name).
- Categories. This table contains income/spending categories (e.g. 
  healthcare, food, etc.). For each category its name and type 
  (income/outcome) is stored.
- Transactions. This is the main table with all the transactions performed.
  For each transaction DB stores bank account and category references as
  long as amount of money gained or spent.
- Plans. This table contains budget plans. Each plan contains name and
  limit of outcomes for a month. Plan is connected to a specific ccategory.

Physical ER-diagram of `libbdgt`'s DB demonstrates some low-level details 
such as encrypted columns (of type `bytea`) and is shown below.

![Physical ER-diagram](../pictures/er-physical.drawio.png)
