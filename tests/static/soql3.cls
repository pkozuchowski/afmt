@isTest
private class UTIL_SortContact_TEST {
  @isTest
  private static void sortContactsLastFirstName() {
    queryLocator = System.Database.getQueryLocator([SELECT Id FROM Log__c WHERE (LogRetentionDate__c <= :LOG_RETENTION_END_DATE AND LogRetentionDate__c != NULL) OR TotalLogEntries__c = 0 ORDER BY LogRetentionDate__c]);
  }
}