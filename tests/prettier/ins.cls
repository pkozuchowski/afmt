public class Me {
  {
    SObject a = new Account();

    if (a instanceof Account) {
      Account = (Account) a;
    } else {
      // something else...
    }
  }
}