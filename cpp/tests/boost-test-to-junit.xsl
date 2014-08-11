<?xml version="1.0" encoding="utf-8"?>

<!--
A stylesheet for converting the XML output of Boost.Test to Junit format.

Other solutions are listed at http://stackoverflow.com/questions/2969131/anyone-have-an-xsl-to-convert-boost-test-xml-logs-to-a-presentable-format
In particular, http://stackoverflow.com/a/3407178, looks better because it integrates TestLog into results..
But even after some modification, I could not get it to work.
-->

<xsl:stylesheet version="1.0"
                xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
                xmlns:msxsl="urn:schemas-microsoft-com:xslt"
                exclude-result-prefixes="msxsl">

  <xsl:output method="xml"
              indent="yes"/>

  <xsl:template match="TestLog">
  </xsl:template>
  <xsl:template match="TestingTime">
  </xsl:template>
  <xsl:template match="Error">
  </xsl:template>
  
   <xsl:template match="TestResult">
      <xsl:call-template name="testSuite" />
  </xsl:template>

  <xsl:template name="testSuite">
    <xsl:for-each select="TestSuite">
      <testsuite>
        <xsl:attribute name="errors">
            <xsl:value-of select="@test_cases_failed + @test_cases_aborted"/>
        </xsl:attribute>

        <xsl:attribute name="tests">
            <xsl:value-of select="@test_cases_passed + @test_cases_failed + @test_cases_skipped + @test_cases_aborted"/>
        </xsl:attribute>

        <xsl:attribute name="skipped">
            <xsl:value-of select="@test_cases_skipped"/>
        </xsl:attribute>

        <xsl:attribute name="failures">
            <xsl:value-of select="@test_cases_failed"/>
        </xsl:attribute>

        <xsl:call-template name="testAttributes" />
        <!--results-->
          <xsl:call-template name="testSuite" />
          <xsl:for-each select="TestCase">
            <testcase>
              <xsl:call-template name="testAttributes" />
              <xsl:call-template name="testCaseElements" />
            </testcase>
          </xsl:for-each>
        <!--/results-->
      </testsuite>
    </xsl:for-each>
  </xsl:template>

  <xsl:template name="testAttributes">
    <xsl:attribute name="name">
      <xsl:value-of select="@name"/>
    </xsl:attribute>
    <xsl:attribute name="success">
      <xsl:choose>
        <xsl:when test="@result = 'passed'">True</xsl:when>
        <xsl:when test="@result != 'passed'">False</xsl:when>
      </xsl:choose>
    </xsl:attribute>
    <xsl:attribute name="executed">True</xsl:attribute>
    <xsl:attribute name="time">0</xsl:attribute>
    <xsl:attribute name="asserts">
      <xsl:value-of select="@assertions_failed + @assertions_passed"/>
    </xsl:attribute>
  </xsl:template>

  <xsl:template name="testCaseElements">
    <xsl:if test="@result != 'passed'">
        <failure type="No type reported" message="No message reported"/>
    </xsl:if>
  </xsl:template>

</xsl:stylesheet>
