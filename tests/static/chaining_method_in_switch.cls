class A {
  {
    switch on
      SwitchCaseGenerator
        .setType('Switch Type')
        .setMessage('The Most Important Message')
        .setSeverity('High')
        .getInstance() {
      when 1 {
        System.debug('1');
      }
      when else {
        System.debug('Else');
      }
    }
  }
}