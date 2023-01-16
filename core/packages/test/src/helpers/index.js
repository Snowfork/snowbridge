// Sleep is a wait function
const sleep = async (ms) => new Promise(resolve => setTimeout(resolve, ms));

const granularity = (decimals) => Math.pow(10, 18 - decimals);

const wrapped = (amount, decimals) =>
  amount.multipliedBy(granularity(decimals));

const unwrapped = (amount, decimals) =>
  amount.dividedToIntegerBy(granularity(decimals));

module.exports = {
    sleep, wrapped, unwrapped
};
