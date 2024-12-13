public class Me {
  {
    SObject s = new Account();

    switch on s {
      when Account a {
        System.debug('account ' + a);
      }
      when Contact c {
        System.debug('contact ' + c);
      }
      when null {
        System.debug('null');
      }
      when else {
        System.debug('default');
      }
    }
  }
}