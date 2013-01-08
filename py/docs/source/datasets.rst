Datasets
====================================

.. py:class:: Dataset

    A dataset
    
    .. py:method:: save([path,backup])
    
        Save the dataset.
        
        :param path: path to save to
        :param backup: whether this is a backup or not
    
    .. py:method:: tables()
    
        Get a list of tables in the dataset.

    .. py:method:: indices([table])
    
        Get a list of indices in the dataset.
        
        :param table: only list indices for this table
