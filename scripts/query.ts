let try_query_count = async () => {
  
    const my_query = await secretjs.query.compute.queryContract({
      contract_address: contract_address,
      code_hash: contract_code_hash,
      query: { get_state: {} },
    });
  
    console.log(my_query);
};
  
  // try_query_count();
  const {
    balance: { amount },
  } = await secretjs.query.bank.balance(
    {
      address: wallet.address,
      denom: "uscrt",
    } /*,
    // optional: query at a specific height (using an archive node) 
    [["x-cosmos-block-height", "2000000"]]
    */,
  );
  console.log(amount);