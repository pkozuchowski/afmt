class SoqlHaving {
  {
    [
      SELECT count(Id)
      FROM Lead
      GROUP BY LeadSource
      HAVING COUNT(Name) > 100 AND LeadSource LIKE 'San%'
    ];
  }
}