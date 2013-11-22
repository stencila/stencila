Tables and Tablesets
====================================

.. py:class:: Table

    A data table
    
    .. py:method:: rows()
    
        Get the number of rows in the table.
    
    .. py:method:: columns()
    
        Get the number of columns in the table.
    
    .. py:method:: labels()
    
        Get the labels of the columns in the table.


.. py:class:: Tableset

    A set of tables
    
    .. py:method:: save([path,backup])
    
        Save the Tableset.
        
        :param path: path to save to
        :param backup: whether this is a backup or not
    
    .. py:method:: tables()
    
        Get a list of tables in the Tableset.

    .. py:method:: indices([table])
    
        Get a list of indices in the Tableset.
        
        :param table: only list indices for this table
