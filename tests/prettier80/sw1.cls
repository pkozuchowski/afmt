public class Me {
  {
    Integer i = 0;

    switch on i {
      when 2 {
        System.debug('when block 2');
      }
      when null {
        System.debug('bad integer');
      }
      when else {
        System.debug('default ' + i);
      }
    }
  }
}