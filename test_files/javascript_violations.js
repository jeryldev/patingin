// JavaScript violation examples

function userService() {
  console.log('Starting user service'); // MAJOR violation
  
  function processData(user, auth, config, meta, options, callback, debug) {
    // Long parameter list (MAJOR violation)
    console.log('Processing:', user); // Another violation
    return callback(null, user);
  }
  
  // Good example (no violations)
  function safeProcessData(userData) {
    // Use proper logging instead of console.log
    return { result: userData };
  }
  
  return {
    processData,
    safeProcessData
  };
}

module.exports = userService;
