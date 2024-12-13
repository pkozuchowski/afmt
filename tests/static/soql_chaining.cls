class Chaining {
  {
    [SELECT Id, Name FROM acc].a;

    String g1 = [
      SELECT Id, Name, One_Field__c, Another_Field__c
      FROM Account
      WHERE Id = :accId
    ];

    String g2 = [SELECT Id FROM Account].a;

    [
      SELECT Id, Name, One_Field__c, Another_Field__c
      FROM Account
      WHERE Id = :accId
    ]
      .Name()
      .bbbbbbbbbbbbbbbbbbbbb
      .cccccccccccccccccc
      .aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa();

    String g3 =
      [
        SELECT Id, Name, One_Field__c, Another_Field__c
        FROM Account
        WHERE Id = :accId
      ]
        .Name()
        .bbbbbbbbbbbbbbbbbbbbb
        .cccccccccccccccccc
        .aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa();

    String g3 =
      [
        SELECT Id, Name, One_Field__c, Another_Field__c
        FROM Account
        WHERE Id = :accId
      ]
        .Name;
  }
}