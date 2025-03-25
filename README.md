###### PROGRAM FLOW

<Admin Initializes The Token Vaults, and Adds Each To Global Registry of Whitelisted Tokens, and Their Vaults.>

<Lender

- Lender will first make a deposit ( the amount of tokens the lender wishes to lend), and submit loan terms
- There would be a validation of the specified loan terms that ensures that it is standardized.
- A Lender position PDA ought to be created.
- Lender should be able to withdraw the interest earned.
- Lender should be able to cancel lending order, if no matching is found for a specified deadline.

* Between When Lender Submits a Lending Order and When A Borrower Is Matched, the lent tokens would just be sitting
  idly in the Token Vault; That is Not Capital Efficient.

- for That reason, Until lender's order is matched, lender's lent tokens would be used in staking so lender can earn additional rewards

* Specifies loan terms including the following struct:
  { + interest rate. + lending duration
  }>
