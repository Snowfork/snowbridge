// Sleep is a wait function
const sleep = async (ms) => new Promise(resolve => setTimeout(resolve, ms));

module.exports = {
    sleep
};
