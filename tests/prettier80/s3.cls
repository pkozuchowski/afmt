public class Me {
  {
    String name = [SELECT Name FROM Account WHERE Id = :A.Id][0].Name;
  }
}