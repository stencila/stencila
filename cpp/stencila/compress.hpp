//! @file compress.hpp
//! @brief Classes and functions for working with compressed file archives
//! @author Nokome Bentley

#pragma once

#include <vector>
#include <string>
#include <iostream>
#include <ctime>
#include <fstream>

#include <boost/filesystem.hpp>

#include <libarchive/archive.h>
#include <libarchive/archive_entry.h>

#include <stencila/exception.hpp>

namespace Stencila {
namespace Compress {

class Writer {

private:

    archive* archive_;

public:

    typedef std::string String;
    
    Writer(const String& path){
        archive_ = archive_write_new();
        archive_write_set_format_ustar(archive_);
        archive_write_add_filter_gzip(archive_);
        archive_write_open_filename(archive_,path.c_str());
    }
    
    ~Writer(void){
        archive_write_free(archive_);
    }

    void set(const String& path,const String& content){
        archive_entry* entry = archive_entry_new();
        archive_entry_set_pathname(entry, path.c_str());
        archive_entry_set_size(entry, content.length());
        archive_entry_set_filetype(entry, AE_IFREG);
        archive_entry_set_mtime(entry, std::time(0), 0);
        archive_entry_set_perm(entry, 0740);
        archive_write_header(archive_, entry);
        archive_write_data(archive_, content.c_str(), content.length());
        archive_write_finish_entry(archive_);
        archive_entry_free(entry);
    }
    
    void add(const String& to,const String& from){
        
        std::ifstream file(from);
        
        //Get file size
        file.seekg(0,std::ios::end);
        std::streampos size = file.tellg();
        file.seekg(0,std::ios::beg);
        
        archive_entry* entry = archive_entry_new();
        archive_entry_set_pathname(entry,to.c_str());
        archive_entry_set_size(entry, size);
        archive_entry_set_filetype(entry, AE_IFREG);
        archive_entry_set_mtime(entry, std::time(0), 0);
        archive_entry_set_perm(entry, 0740);
        archive_write_header(archive_, entry);
        
        char buffer[8192];
        while(file.good()){
            file.read(buffer, sizeof(buffer));
            archive_write_data(archive_, buffer, file.gcount());
        }
        file.close();
        archive_entry_free(entry);
    }
    
    void close(void){
        int code  = archive_write_close(archive_);
        if(code != ARCHIVE_OK) STENCILA_THROW(Exception,archive_error_string(archive_));
    }
};


class Reader {

private:

    archive* archive_;

public:

    typedef std::string String;
    
    Reader(const String& path){
        archive_ = archive_read_new();
        archive_read_support_format_tar(archive_);
        archive_read_support_filter_gzip(archive_);
        if(archive_read_open_filename(archive_,path.c_str(),10240)!=ARCHIVE_OK) STENCILA_THROW(Exception,archive_error_string(archive_));
    }
    
    ~Reader(void){
        archive_read_free(archive_);
    }

    String get(const String& path){
        std::string content;
        archive_entry* entry;
        while (archive_read_next_header(archive_,&entry)==ARCHIVE_OK) {
          if(archive_entry_pathname(entry)==path){
              while(true){
                char buffer[1000];
                int read = archive_read_data(archive_,buffer,1000);
                if(read>0) content += std::string(buffer).substr(0,read);
                else if(read==0) break;
              }
              break;
          }
        }
        return content;
    }
    
    void extract(const String& from,const String& to){
        boost::filesystem::create_directories(to);
        while(true){
            archive_entry* entry;
            int code1 = archive_read_next_header(archive_,&entry);
            if(code1 == ARCHIVE_EOF) break;
            if(code1 != ARCHIVE_OK) STENCILA_THROW(Exception,archive_error_string(archive_))
            
            archive_entry_set_pathname(entry,(to+"/"+archive_entry_pathname(entry)).c_str());
            
            int code2 = archive_read_extract(archive_,entry,ARCHIVE_EXTRACT_TIME|ARCHIVE_EXTRACT_PERM);
            if(code2 != ARCHIVE_OK) STENCILA_THROW(Exception,archive_error_string(archive_))
        }
    }
    
};


}
}
